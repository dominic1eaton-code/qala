import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-workspaces',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './workspaces.component.html',
  styleUrl: './workspaces.component.css'
})
export class FactoryWorkspacesComponent {}
