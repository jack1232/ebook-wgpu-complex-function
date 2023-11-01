#![allow(dead_code)]
use num_complex::{Complex, ComplexFloat};
use super::colormap;

#[derive(Default)]
pub struct IComplex3DOutput {
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

pub struct IComplex3D {
    xmin: f32,
    xmax: f32,
    zmin: f32,
    zmax: f32,
    pub func_selection: u32,
    pub x_resolution: u32,
    pub z_resolution: u32,
    pub scale: f32,
    pub aspect_ratio: f32,
    pub colormap_name: String,
    pub t: f32,  // animation time parameter
}

impl Default for IComplex3D {
    fn default() -> Self {
        Self {
            func_selection: 0,
            xmin: -2.0,
            xmax: 2.0,
            zmin: -2.0,
            zmax: 2.0,
            x_resolution: 101,
            z_resolution: 101,
            scale: 1.0,
            aspect_ratio: 1.0,
            colormap_name: "jet".to_string(),
            t: 0.0,
        }
    }
}

impl IComplex3D {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn create_complex_data(&mut self) -> IComplex3DOutput {
        let mut positions:Vec<[f32; 3]> = vec![];
        let mut colors:Vec<[f32; 3]> = vec![];
        let cdr = self.complex_data_range();

        let cdata = colormap::colormap_data(&self.colormap_name);

        for i in 0..=self.x_resolution as usize{
            for j in 0..=self.z_resolution as usize {
                positions.push(cdr.0[i][j]);
                let color = colormap::color_lerp(cdata, cdr.2[0], cdr.2[1], cdr.1[i][j]);
                colors.push(color);
            }
        }

        // calculate indices
        let mut indices:Vec<u32> = vec![];
        let vertices_per_row = self.z_resolution + 1;
        for i in 0..self.x_resolution {
            for j in 0..self.z_resolution {
                let idx0 = j + i * vertices_per_row;
                let idx1 = j + 1 + i * vertices_per_row;
                let idx2 = j + 1 + (i + 1) * vertices_per_row;
                let idx3 = j + (i + 1) * vertices_per_row; 

                let values = vec![idx0, idx1, idx2, idx2, idx3, idx0];
                indices.extend(values);
            }
        }
        IComplex3DOutput { positions, colors, indices }
    }


    fn complex_data_range(&mut self) -> (Vec<Vec<[f32;3]>>, Vec<Vec<f32>>, [f32; 2]) {
        let dx = (self.xmax - self.xmin)/self.x_resolution as f32;
        let dz = (self.zmax - self.zmin)/self.z_resolution as f32;

        let (mut cmin, mut cmax) = (std::f32::MAX, std::f32::MIN);
        let (mut ymin, mut ymax) = (std::f32::MAX, std::f32::MIN);

        let mut pts:Vec<Vec<[f32;3]>> = vec![];
        let mut cps:Vec<Vec<f32>> = vec![];
       
        for i in 0..=self.x_resolution {
            let x = self.xmin + dx * i as f32;
            let mut pt1:Vec<[f32; 3]> = vec![];
            let mut cp1:Vec<f32> = vec![];
            for j in 0..=self.z_resolution {
                let z = self.zmin + dz * j as f32;
                let pt = self.complex_func(x, z);
                //let pt = pp.0;
                pt1.push(pt.0);
                cp1.push(pt.1[1]);

                ymin = if pt.0[1] < ymin { pt.0[1] } else { ymin };
                ymax = if pt.0[1] > ymax { pt.0[1] } else { ymax };
                cmin = if pt.1[1] < cmin { pt.1[1] } else { cmin };
                cmax = if pt.1[1] > cmax { pt.1[1] } else { cmax };
            }
            pts.push(pt1);
            cps.push(cp1);
        }

        for i in 0..=self.x_resolution as usize {
            for j in 0..=self.z_resolution as usize {
                pts[i][j] = self.normalize_point(pts[i][j], ymin, ymax);
            }
        }

        (pts, cps, [cmin, cmax])
    } 

    fn normalize_point(&mut self, pt:[f32; 3], ymin:f32, ymax:f32) -> [f32; 3] {
        let mut pt1 = [0f32; 3];
        pt1[0] = self.scale * (-1.0 + 2.0 * (pt[0] - self.xmin) / (self.xmax - self.xmin));
        pt1[1] = self.scale * (-1.0 + 2.0 * (pt[1] - ymin) / (ymax - ymin)) * self.aspect_ratio;
        pt1[2] = self.scale * (-1.0 + 2.0 * (pt[2] - self.zmin) / (self.zmax - self.zmin));
        pt1
    }

    fn complex_func(&mut self, x:f32, y:f32) -> ([f32; 3], [f32; 3]) {
        let z = Complex::new(x, y);
        let mut fz = z;
        
        let func_select = self.func_selection;
        let t = self.t;

        if func_select == 0 {
            fz = (z - t)/(z*z + z + t);
            (self.xmin, self.xmax, self.zmin, self.zmax) = (-3.0, 2.0, -2.0, 2.0); 
        } else if func_select == 1 {
            let f1 = Complex::new(-z.im - 3.0 * t, z.re);
            let f2 = Complex::new(-z.im + t, z.re);
            fz = (f1.ln()/f2.ln()).sqrt();
            (self.xmin, self.xmax, self.zmin, self.zmax) = (-6.0, 6.0, -6.0, 6.0);
        } else if func_select == 2 {
            fz = t * (t * z).sin();
            (self.xmin, self.xmax, self.zmin, self.zmax) = (-6.0, 6.0, -6.0, 6.0);
        } else if func_select == 3 {
            fz = (0.5 + t) * ((0.5 + t) * z).tan().tan();
            (self.xmin, self.xmax, self.zmin, self.zmax) = (-10.0, 10.0, -1.0, 1.0);
        } else if func_select == 4 {
            fz = t * ((0.5 + t) *z).sin().tan();
            (self.xmin, self.xmax, self.zmin, self.zmax) = (-8.0, 8.0, -2.0, 2.0);
        } else if func_select == 5 {
            let f1 = Complex::new(t + z.re, z.im);
            let f2 = Complex::new(t - z.re, -z.im);
            fz = f1.sqrt() + f2.sqrt();
            (self.xmin, self.xmax, self.zmin, self.zmax) = (-2.0, 2.0, -2.0, 2.0);
        } else if func_select == 6 {
            fz = ((0.5 + t) * z).exp().tan()/z;
            (self.xmin, self.xmax, self.zmin, self.zmax) = (-1.0, 2.0, -1.0, 1.0);
        } else if func_select == 7 {
            fz = ((0.5 + t) * z).sin().cos().sin()/(z*z - t);
            (self.xmin, self.xmax, self.zmin, self.zmax) = (-2.0, 2.0, -1.0, 1.0);
        } else if func_select == 8 {
            let f1 = 0.5 + t;
            let f2 = 1.0 + ((0.5 + t)*z).powi(5);
            fz = f1/f2;
            (self.xmin, self.xmax, self.zmin, self.zmax) = (-1.0, 1.0, -1.0, 1.0);
        } else if func_select == 9 {
            let f1 = ((0.5 + t) * z).sin();
            let f2 = ((0.5 + t) * z).exp().cos() *(z * z - (0.5 + t)*(0.5 + t));
            fz = f1/f2;
            (self.xmin, self.xmax, self.zmin, self.zmax) = (-4.0, 6.0, -2.0, 2.0);
        } else if func_select == 10 {
            fz = 1.0/(z + t) + 1.0/(z - t);
            (self.xmin, self.xmax, self.zmin, self.zmax) = (-2.0, 2.0, -2.0, 2.0);
        }

        ([x, fz.abs(), y], [x, fz.arg(), y])
    }
}